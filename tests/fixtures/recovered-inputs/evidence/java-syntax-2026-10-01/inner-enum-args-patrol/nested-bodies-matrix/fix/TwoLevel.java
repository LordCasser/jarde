package p;

public class TwoLevel {
    public static class Middle {
        public enum Deep {
            X {
                @Override
                public int v() { return 1; }
            },
            Y {
                @Override
                public int v() { return 2; }
            };

            public abstract int v();
        }
    }
}
