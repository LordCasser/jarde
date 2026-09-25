// P3 6.6: a field's own `ConstantValue` is its declaration's initializer.
//
//     javac --release 8 -g:none -d v8 Ops2.java Consts.java
//
// `N`, `L`, `FLAG`, `S`, `D` and `F` each carry the `ConstantValue` attribute the compiler folds
// the initializer into, and `plain` carries none: the field's own attribute table is the fact, and
// no initializer is read off a `<clinit>` or a use site. A `double` or `float` constant's digits
// are a spelling rule of their own (2c.5), so those two keep an initializer-less declaration.
package p;

class Consts {
    static final int N = 3;
    static final long L = 3L;
    static final boolean FLAG = true;
    static final String S = "a";
    static final double D = 0.5;
    static final float F = 0.5f;
    static int plain;
}
