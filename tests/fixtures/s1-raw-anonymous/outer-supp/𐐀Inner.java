package p;
public class 𐐀Inner {
    static Object observed;
    int f = 37;

    Runnable make() {
        return new Runnable() {
            @Override
            public void run() {
                observed = 𐐀Inner.this;
                𐐀Inner.this.f++;
            }
        };
    }

    public static void main(String[] args) {
        𐐀Inner inner = new 𐐀Inner();
        inner.make().run();
        System.out.println(observed.equals(inner));
        System.out.println(inner.f);
    }
}
