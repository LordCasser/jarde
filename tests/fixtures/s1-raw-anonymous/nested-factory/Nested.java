package p;

public final class Nested {
    public static int trace;

    public static 𐐀Factory create() {
        return new 𐐀Factory() {
            @Override
            public Action make() {
                return new Action() {
                    @Override
                    public void run() {
                        trace++;
                    }
                };
            }
        };
    }
}
