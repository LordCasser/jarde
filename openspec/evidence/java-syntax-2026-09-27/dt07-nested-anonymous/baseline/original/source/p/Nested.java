package p;

public final class Nested {
    public static int trace;

    public static Factory create() {
        return new Factory() {
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
