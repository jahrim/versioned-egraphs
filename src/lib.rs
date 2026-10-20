mod lib {
    fn api(){}

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test] fn unit_test() { api(); }
    }
}