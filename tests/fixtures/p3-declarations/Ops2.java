// P3 6.1/6.2/6.5: the package a class's own name states, a member's own `throws`, and an
// interface member's `default`.
//
//     javac --release 8 -g:none -d v8 Ops2.java Consts.java
//
// `plus` declares no `Code` attribute and is the abstract member Java spells `int plus(int, int);`;
// `zero` and `unit` each declare one, and their own flags say which of them is a `default` member
// and which is `static`. `load`'s `throws` clause is the `Exceptions` attribute the compiler writes
// from its declaration and from nothing else — no body of this sample throws a checked exception.
package p;

public interface Ops2 {
    int plus(int a, int b);

    default int zero() {
        return 0;
    }

    static int unit() {
        return 1;
    }

    void load(String path) throws java.io.IOException, java.lang.InterruptedException;
}
