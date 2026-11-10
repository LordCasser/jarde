package p;

public final class Nested {
    public static int trace;

    public static Factory create() {
        return new Factory() {
            @Override
            public 𐐀Action make() {
                return new 𐐀Action() {
                    @Override
                    public void run() {
                        trace++;
                    }
                };
            }
        };
    }
}
