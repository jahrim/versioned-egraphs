// ### DISCLAIMER ##############################################################
// This file is an adaptation of the patterns, substitutions and recursive
// expressions from the egg's:
// https://github.com/egraphs-good/egg/blob/main/src/pattern.rs
// https://github.com/egraphs-good/egg/blob/main/src/subst.rs
// https://github.com/egraphs-good/egg/blob/main/src/language.rs
//
// The implementation has been specialized to our [ENode]s, so that it does not
// depend on egg's `Language`.
//
// egg is Copyright 2019 Max Willsey, licensed under the MIT License.
// See the THIRD-PARTY file at the root of this repository.
// #############################################################################

use super::machine::Program;
use crate::structures::egraph::{EClass, ENode};
use crate::structures::Map;
use crate::util::id::{Id, SymbolIds};
use smallvec::SmallVec;
use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

/// A variable in a [Pattern], written as `?name`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Var(Id);
impl FromStr for Var {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('?') && s.len() > 1 {
            Ok(Var(SymbolIds::from_string(s)))
        } else {
            Err(ParseError::MissingQuestionMark(s.to_owned()))
        }
    }
}
impl Display for Var {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", SymbolIds::to_string(self.0).unwrap_or("?"))
    }
}
impl Debug for Var {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

/// A mapping from [Var]s to [EClass]es.
#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Subst {
    vec: SmallVec<[(Var, EClass); 3]>,
}
impl Subst {
    pub fn with_capacity(capacity: usize) -> Self {
        Self { vec: SmallVec::with_capacity(capacity) }
    }
    /// Bind the [Var] `var` to the [EClass] `id`.
    /// #### Return
    /// The [EClass] previously bound to `var`, if any.
    pub fn insert(&mut self, var: Var, id: EClass) -> Option<EClass> {
        for pair in &mut self.vec {
            if pair.0 == var {
                return Some(std::mem::replace(&mut pair.1, id));
            }
        }
        self.vec.push((var, id));
        None
    }
    /// #### Return
    /// The [EClass] bound to the [Var] `var`, if any.
    pub fn get(&self, var: Var) -> Option<&EClass> {
        self.vec.iter().find_map(|(v, id)| if *v == var { Some(id) } else { None })
    }
    /// #### Return
    /// An iterator over the bindings of this [Subst].
    pub fn iter(&self) -> impl Iterator<Item = &(Var, EClass)> {
        self.vec.iter()
    }
}
impl FromIterator<(Var, EClass)> for Subst {
    fn from_iter<T: IntoIterator<Item = (Var, EClass)>>(iter: T) -> Self {
        Self { vec: iter.into_iter().collect() }
    }
}
impl std::ops::Index<Var> for Subst {
    type Output = EClass;
    fn index(&self, var: Var) -> &Self::Output {
        match self.get(var) {
            Some(id) => id,
            None => panic!("Var '{}' not found in {:?}", var, self),
        }
    }
}
impl Debug for Subst {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.vec.iter().map(|(v, id)| (v, id))).finish()
    }
}

/// A node in a [PatternAst], either an [ENode] or a [Var].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ENodeOrVar {
    /// An [ENode] whose arguments are indices of other nodes in the
    /// enclosing [PatternAst].
    ENode(ENode),
    Var(Var),
}

/// A pattern represented as a flat list of [ENodeOrVar]s, where each node
/// only refers to nodes before it. The last node is the root.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct PatternAst {
    nodes: Vec<ENodeOrVar>,
}
impl PatternAst {
    /// Append the node `node` to this [PatternAst].
    /// #### Return
    /// The index of `node` in this [PatternAst].
    pub fn add(&mut self, node: ENodeOrVar) -> Id {
        debug_assert!(
            match &node {
                ENodeOrVar::ENode(n) => n.args.iter().all(|&a| usize::from(a) < self.nodes.len()),
                ENodeOrVar::Var(_) => true,
            },
            "node {:?} has children not in {:?}",
            node,
            self
        );
        self.nodes.push(node);
        Id::from(self.nodes.len() - 1)
    }
    /// #### Return
    /// The index of the root of this [PatternAst].
    pub fn root(&self) -> Id {
        Id::from(self.nodes.len() - 1)
    }
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &ENodeOrVar> {
        self.nodes.iter()
    }
    /// #### Return
    /// A new [PatternAst] containing only the subpattern rooted at `root`,
    /// where structurally equal subpatterns are shared.
    pub fn extract(&self, root: Id) -> PatternAst {
        fn aux(
            src: &PatternAst,
            id: Id,
            dst: &mut PatternAst,
            memo: &mut Map<ENodeOrVar, Id>,
        ) -> Id {
            let node = match &src[id] {
                ENodeOrVar::Var(v) => ENodeOrVar::Var(*v),
                ENodeOrVar::ENode(n) => ENodeOrVar::ENode(n.map_args(|a| aux(src, a, dst, memo))),
            };
            *memo.entry(node.clone()).or_insert_with(|| dst.add(node))
        }
        let mut extracted = PatternAst::default();
        aux(self, root, &mut extracted, &mut Default::default());
        extracted
    }
}
impl std::ops::Index<Id> for PatternAst {
    type Output = ENodeOrVar;
    fn index(&self, id: Id) -> &Self::Output {
        &self.nodes[usize::from(id)]
    }
}
impl<'a> IntoIterator for &'a PatternAst {
    type Item = &'a ENodeOrVar;
    type IntoIter = std::slice::Iter<'a, ENodeOrVar>;
    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter()
    }
}
impl Display for PatternAst {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fn aux(ast: &PatternAst, id: Id, f: &mut Formatter<'_>) -> std::fmt::Result {
            match &ast[id] {
                ENodeOrVar::Var(v) => write!(f, "{}", v),
                ENodeOrVar::ENode(n) if n.args.is_empty() => write!(f, "{}", n),
                ENodeOrVar::ENode(n) => {
                    write!(f, "({}", n)?;
                    for &arg in &n.args {
                        write!(f, " ")?;
                        aux(ast, arg, f)?;
                    }
                    write!(f, ")")
                },
            }
        }
        if self.nodes.is_empty() {
            write!(f, "()")
        } else {
            aux(self, self.root(), f)
        }
    }
}
impl FromStr for PatternAst {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens = tokenize(s);
        let mut tokens = tokens.iter().copied().peekable();
        let mut ast = PatternAst::default();
        parse_sexp(&mut tokens, &mut ast)?;
        match tokens.next() {
            None => Ok(ast),
            Some(token) => Err(ParseError::UnexpectedToken(token.to_owned())),
        }
    }
}

fn tokenize(s: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut start = None;
    for (i, c) in s.char_indices() {
        if c == '(' || c == ')' || c.is_whitespace() {
            if let Some(j) = start.take() {
                tokens.push(&s[j..i]);
            }
            if !c.is_whitespace() {
                tokens.push(&s[i..i + 1]);
            }
        } else if start.is_none() {
            start = Some(i);
        }
    }
    if let Some(j) = start {
        tokens.push(&s[j..]);
    }
    tokens
}

fn parse_sexp<'a>(
    tokens: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
    ast: &mut PatternAst,
) -> Result<Id, ParseError> {
    match tokens.next() {
        None => Err(ParseError::EmptySexp),
        Some(")") => Err(ParseError::UnexpectedToken(")".to_owned())),
        Some("(") => {
            let op = match tokens.next() {
                None => return Err(ParseError::UnclosedList),
                Some(")") => return Err(ParseError::EmptySexp),
                Some("(") => return Err(ParseError::HeadList),
                Some(op) => op,
            };
            let mut args = Vec::new();
            loop {
                match tokens.peek() {
                    None => return Err(ParseError::UnclosedList),
                    Some(&")") => {
                        tokens.next();
                        break;
                    },
                    Some(_) => args.push(parse_sexp(tokens, ast)?),
                }
            }
            Ok(ast.add(from_op(op, args)?))
        },
        Some(op) => Ok(ast.add(from_op(op, vec![])?)),
    }
}

fn from_op(op: &str, args: Vec<Id>) -> Result<ENodeOrVar, ParseError> {
    if op.starts_with('?') && op.len() > 1 {
        if args.is_empty() {
            op.parse().map(ENodeOrVar::Var)
        } else {
            Err(ParseError::UnexpectedVar(op.to_owned()))
        }
    } else {
        Ok(ENodeOrVar::ENode(ENode::application(SymbolIds::from_string(op), args)))
    }
}

/// An error raised when parsing a [Pattern].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    EmptySexp,
    UnclosedList,
    HeadList,
    UnexpectedToken(String),
    UnexpectedVar(String),
    MissingQuestionMark(String),
}
impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::EmptySexp => write!(f, "found empty s-expression"),
            ParseError::UnclosedList => write!(f, "found unclosed list"),
            ParseError::HeadList => write!(f, "found a list in the head position"),
            ParseError::UnexpectedToken(t) => write!(f, "found unexpected token {:?}", t),
            ParseError::UnexpectedVar(v) => write!(f, "found variable {:?} applied to arguments", v),
            ParseError::MissingQuestionMark(v) => {
                write!(f, "pattern variable {:?} should have a leading question mark", v)
            },
        }
    }
}
impl std::error::Error for ParseError {}

/// A pattern that can be matched against an [EGraph](crate::structures::egraph::EGraph).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pattern {
    pub ast: PatternAst,
    pub(crate) program: Program,
}
impl Pattern {
    pub fn new(ast: PatternAst) -> Self {
        let program = Program::compile_from_pat(&ast);
        Pattern { ast, program }
    }
}
impl From<PatternAst> for Pattern {
    fn from(ast: PatternAst) -> Self {
        Pattern::new(ast)
    }
}
impl FromStr for Pattern {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        PatternAst::from_str(s).map(Pattern::new)
    }
}
impl Display for Pattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.ast, f)
    }
}

/// The matches of a [Pattern] within an [EClass].
#[derive(Debug)]
pub struct SearchMatches<'a> {
    /// The [EClass] where the matches were found.
    pub eclass: EClass,
    /// The substitutions for each match.
    pub substs: Vec<Subst>,
    /// The [PatternAst] of the matched [Pattern].
    pub ast: Option<Cow<'a, PatternAst>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_display() {
        for s in ["0", "?x", "(add 2 2)", "(add ?x (add ?y ?y))", "(f (g ?x) (g ?x))"] {
            assert_eq!(s.parse::<PatternAst>().unwrap().to_string(), s);
        }
        assert_eq!("(0)".parse::<PatternAst>().unwrap().to_string(), "0");
        assert_eq!("(?x)".parse::<PatternAst>().unwrap().to_string(), "?x");
        assert_eq!("  ( add\n?x  0 ) ".parse::<PatternAst>().unwrap().to_string(), "(add ?x 0)");
    }

    #[test]
    fn parse_errors() {
        assert_eq!("".parse::<PatternAst>(), Err(ParseError::EmptySexp));
        assert_eq!("()".parse::<PatternAst>(), Err(ParseError::EmptySexp));
        assert_eq!("(add ?x".parse::<PatternAst>(), Err(ParseError::UnclosedList));
        assert_eq!("((add) ?x)".parse::<PatternAst>(), Err(ParseError::HeadList));
        assert_eq!("(add ?x))".parse::<PatternAst>(), Err(ParseError::UnexpectedToken(")".into())));
        assert_eq!("?x ?y".parse::<PatternAst>(), Err(ParseError::UnexpectedToken("?y".into())));
        assert_eq!("(?x 0)".parse::<PatternAst>(), Err(ParseError::UnexpectedVar("?x".into())));
        assert_eq!("x".parse::<Var>(), Err(ParseError::MissingQuestionMark("x".into())));
    }

    #[test]
    fn extract_shares_equal_subpatterns() {
        let ast: PatternAst = "(f (g ?x) (g ?x))".parse().unwrap();
        assert_eq!(ast.len(), 5);
        let extracted = ast.extract(ast.root());
        assert_eq!(extracted.len(), 3);
        assert_eq!(extracted.to_string(), "(f (g ?x) (g ?x))");
    }

    #[test]
    fn subst() {
        let [x, y] = ["?x", "?y"].map(|v| v.parse::<Var>().unwrap());
        let mut subst = Subst::default();
        assert_eq!(subst.insert(x, Id::from(1)), None);
        assert_eq!(subst.insert(y, Id::from(2)), None);
        assert_eq!(subst.insert(x, Id::from(3)), Some(Id::from(1)));
        assert_eq!(subst[x], Id::from(3));
        assert_eq!(subst[y], Id::from(2));
        assert_eq!(subst.get("?z".parse().unwrap()), None);
    }
}
