public class Inner {
    static Object observed;
    int f = 37;

    Runnable make() {
        return new Runnable() {
            @Override
            public void run() {
                observed = Inner.this;
                Inner.this.f++;
            }
        };
    }

    public static void main(String[] args) {
        Inner inner = new Inner();
        inner.make().run();
        System.out.println(observed == inner);
        System.out.println(inner.f);
    }
}
