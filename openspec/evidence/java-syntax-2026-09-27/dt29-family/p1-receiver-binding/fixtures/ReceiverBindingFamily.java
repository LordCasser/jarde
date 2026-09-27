package dt29p1;

public class ReceiverBindingFamily {
    public static class A {
        public boolean publicField;
        protected boolean protectedField;
        boolean packageField;
        private boolean privateField;

        public boolean all() {
            return publicField && protectedField && packageField && privateField;
        }
    }

    public static class B extends A {
        // These declarations make an uncast B receiver observably wrong.
        public boolean publicField;
        protected boolean protectedField;
        boolean packageField;
        private boolean privateField;

        public void self(boolean value) {
            ((A) this).publicField = value;
            ((A) this).protectedField = value;
            ((A) this).packageField = value;
            ((A) this).privateField = value;
        }

        public boolean hidden() {
            return publicField || protectedField || packageField || privateField;
        }
    }

    public static class C {
        public void set(B receiver, boolean value) {
            ((A) receiver).publicField = value;
            ((A) receiver).protectedField = value;
            ((A) receiver).packageField = value;
            ((A) receiver).privateField = value;
        }
    }

    public static class D {
        public void set(B receiver, boolean value) {
            ((A) receiver).publicField = value;
            ((A) receiver).protectedField = value;
            ((A) receiver).packageField = value;
            ((A) receiver).privateField = value;
        }
    }

    public static int run() {
        B receiver = new B();
        receiver.self(true);
        int first = bits((A) receiver);
        new C().set(receiver, false);
        int second = bits((A) receiver);
        new D().set(receiver, true);
        return first * 100 + second * 10 + bits((A) receiver);
    }

    private static int bits(A receiver) {
        return receiver.publicField ? 1 : 0;
    }

    private static int bits(B receiver) {
        return 9;
    }
}
