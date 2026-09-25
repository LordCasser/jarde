# Mixed short-circuit int return control

`javac --release 8 -g:none MixedIntReturn.java` produces the committed class (SHA-256 `c30697a78bc0d86acb823b3cb73cece1e78a567c68dcf360548419b1da5a6d5d`). Its `value(Z)I` has the same tests at BCI 1/7/13, constant producers at 16/20 and `ireturn` at 21 as the Boolean fixture. The method descriptor alone changes the consumer type proof.
