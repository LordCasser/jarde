/// The depth boundary of `new@1`'s nested-construction scan, as this change moves it: the anchor's
/// own chain is **three** layers, and a **four**-layer run keeps the outermost refusal the patrol
/// registered.
///
/// Both methods are written so that the outermost construction is the only reader of the chain:
/// `threeLayer` presents as one `new` expression, `fourLayer` stays refused at its outermost site.
public final class NestedDepth {
    private NestedDepth() {}

    static final class Third {
        final String s;

        Third(String s) {
            this.s = s;
        }
    }

    static final class Second {
        final Third inner;

        Second(Third inner) {
            this.inner = inner;
        }
    }

    static final class First {
        final Second inner;

        First(Second inner) {
            this.inner = inner;
        }
    }

    static final class Fourth {
        final First inner;

        Fourth(First inner) {
            this.inner = inner;
        }
    }

    public static String threeLayer() {
        return new First(new Second(new Third("t"))).inner.inner.s;
    }

    public static String fourLayer() {
        return new Fourth(new First(new Second(new Third("f")))).inner.inner.inner.s;
    }

    public static void main(String[] args) {
        System.out.println(threeLayer());
        System.out.println(fourLayer());
    }
}
