# Table of Bitwuzla BV rewrite rules

[Source](https://github.com/codydresel/bitwuzla/blob/main/src/rewrite/rewrites_bv.cpp)

| LHS                                                                                                                                    | RHS                                                |
|----------------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------|
| `(bvadd (_ bv0 N) a)` <br> `(bvadd a (_ bv0 N))`                                                                                       | `a`                                                |
| `(bvadd (_ bvX N) (bvadd (_ bvY N) a))`                                                                                                | `(bvadd (_ bvZ N) a)`, Z = X + Y                   |
| `(bvadd a b)` with bw == 1                                                                                                             | `(bvxor a b)`                                      |
| `(bvadd a a)`                                                                                                                          | `(bvmul a (_ bv2 N))`                              |
| `(bvadd a (bvnot a))`                                                                                                                  | `(bvnot (_ bv0 N))`                                |
| `(bvadd a (bvneg a))` <br> `(bvadd (bvneg a) a)`                                                                                       | `0`                                                |
| `(bvadd a (bvneg (bvmul (bvudiv a b) b)))`<br>`(bvadd a (bvmul (bvneg (bvudiv a b)) b))`<br>`(bvadd a (bvmul (bvudiv a b) (bvneg b)))` | `(bvurem a b)`                                     |
| `(bvadd a (bvneg (bvmul b (bvsdiv a b))))`<br>`(bvadd a (bvmul (bvneg b) (bvsdiv a b)))`<br>`(bvadd a (bvmul b (bvneg (bvsdiv a b))))` | `(bvsrem a b)`                                     |
| `(bvadd a (ite c 0 e))` or `(bvadd a (ite c t 0))`                                                                                     | `(ite c a (bvadd e a))` or `(ite c (bvadd t a) a)` |
| `(bvadd a (bvshl b a))`                                                                                                                | `(bvor a (bvshl b a))`                             |
| `(bvneg (bvadd a (bvmul a b)))`                                                                                                        | `(bvmul a (bvnot b))`                              |
