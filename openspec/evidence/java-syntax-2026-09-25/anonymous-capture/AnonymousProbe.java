public class AnonymousProbe {
    private int state = 2;
    public interface Action { int run(int x); }
    public Action make(int base) {
        int captured = base + 1;
        return new Action() {
            @Override public int run(int x) { return captured + state + x; }
        };
    }
    public static void main(String[] args) {
        System.out.println(new AnonymousProbe().make(3).run(4));
    }
}
