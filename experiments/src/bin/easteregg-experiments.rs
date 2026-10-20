fn main() {
    use easter_egg::*;

    // #########################################################################
    // Taken from egg and adapted to easter egg
    // (egg is Copyright 2019 Max Willsey, MIT License; see THIRD-PARTY)
    define_language! {
        enum SimpleMath {
            "+" = Add([Id; 2]),
            "*" = Mul([Id; 2]),
            Num(i32),
            Symbol(Symbol),
        }
    }

    #[derive(Default, Clone)]
    struct ConstantFolding;
    impl Analysis<SimpleMath> for ConstantFolding {
        type Data = Option<i32>;

        fn merge(&self, to: &mut Self::Data, from: Self::Data) -> bool {
            if from > *to {
                *to = from;
                return true;
            }
            false
        }

        fn make(egraph: &EGraph<SimpleMath, Self>, enode: &SimpleMath) -> Self::Data {
            let x = |i: &Id| egraph[*i].data;
            match enode {
                SimpleMath::Num(n) => Some(*n),
                SimpleMath::Add([a, b]) => Some(x(a)? + x(b)?),
                SimpleMath::Mul([a, b]) => Some(x(a)? * x(b)?),
                _ => None,
            }
        }

        fn modify(egraph: &mut EGraph<SimpleMath, Self>, id: Id) {
            let eclass = egraph[id].clone();
            if let Some(i) = eclass.data {
                let added = egraph.colored_add(eclass.color().unwrap(), SimpleMath::Num(i));
                egraph.colored_union(eclass.color().unwrap(), id, added);
            }
        }
    }
    // #########################################################################

    let mut runner = Runner::<SimpleMath, ConstantFolding, ()>::default();
    let v0 = runner.egraph.create_color(None);
    let x = runner.egraph.colored_add(v0, SimpleMath::Symbol(Symbol::from("x")));
    let x_prime = runner.egraph.colored_add(v0, SimpleMath::Symbol(Symbol::from("x")));
    let zero = runner.egraph.colored_add(v0, SimpleMath::Num(0));
    let one = runner.egraph.colored_add(v0, SimpleMath::Num(1));
    let two = runner.egraph.colored_add(v0, SimpleMath::Num(2));

    println!("x = 2 in v0");
    runner.egraph.colored_union(v0, x, two);

    let v01 = runner.egraph.create_color(Some(v0));
    let x01 = runner.egraph.colored_add(v01, SimpleMath::Symbol(Symbol::from("x")));
    let zero01 = runner.egraph.colored_add(v01, SimpleMath::Num(0));

    println!("x = 0 in v0.1");
    runner.egraph.colored_union(v01, x, zero01);

    let v02 = runner.egraph.create_color(Some(v0));
    let x02 = runner.egraph.colored_add(v02, SimpleMath::Symbol(Symbol::from("x")));
    let one02 = runner.egraph.colored_add(v02, SimpleMath::Num(1));

    println!("x = 1 in v0.2");
    runner.egraph.colored_union(v02, x, one02);

    println!("printing eclasses...");
    println!("eclass x (added in v0): {:?}", x);
    println!("eclass x_prime (added in v0): {:?}", x_prime);
    println!("eclass 0 (added in v0): {:?}", zero);
    println!("eclass 1 (added in v0): {:?}", one);
    println!("eclass 2 (added in v0): {:?}", two);
    println!("eclass x (added in v0.1): {:?}", x01);
    println!("eclass 0 (added in v0.1): {:?}", zero01);
    println!("eclass x (added in v0.2): {:?}", x02);
    println!("eclass 1 (added in v0.2): {:?}", one02);

    println!("querying version v0.1 ...");
    println!("find x (added in v0): {:?}", runner.egraph.colored_find(v01, x));
    println!("find x (added in v0.1): {:?}", runner.egraph.colored_find(v01, x01));
    println!("find x (added in v0.2): {:?}", runner.egraph.colored_find(v01, x02));
    println!("find 0 (added in v0.1): {:?}", runner.egraph.colored_find(v01, zero01));
    println!("find 1 (added in v0.2): {:?}", runner.egraph.colored_find(v01, one02));
    println!("find 2 (added in v0): {:?}", runner.egraph.colored_find(v01, two));

    println!("querying version v0.2");
    println!("find x (added in v0): {:?}", runner.egraph.colored_find(v02, x));
    println!("find x (added in v0.1): {:?}", runner.egraph.colored_find(v02, x01));
    println!("find x (added in v0.2): {:?}", runner.egraph.colored_find(v02, x02));
    println!("find 0 (added in v0.1): {:?}", runner.egraph.colored_find(v02, zero01));
    println!("find 1 (added in v0.2): {:?}", runner.egraph.colored_find(v02, one02));
    println!("find 2 (added in v0): {:?}", runner.egraph.colored_find(v02, two));

    println!("querying datas...");
    println!("data x (added in v0): {:?}", runner.egraph[x].data);
    println!("data x (added in v0.1): {:?}", runner.egraph[x01].data);
    println!("data x (added in v0.2): {:?}", runner.egraph[x02].data);
}
