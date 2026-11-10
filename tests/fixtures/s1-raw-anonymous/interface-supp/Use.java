package p;
public class Use {
    static Object observed;
    int f = 37;

    𐐀Action make() {
        return new 𐐀Action() {
            @Override
            public void run() {
                observed = Use.this;
                Use.this.f++;
            }
        };
    }

    public static void main(String[] args) {
        Use inner = new Use();
        inner.make().run();
        System.out.println(observed.equals(inner));
        System.out.println(inner.f);
    }
}
