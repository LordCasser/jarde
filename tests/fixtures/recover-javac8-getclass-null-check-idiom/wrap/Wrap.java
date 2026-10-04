public class Wrap {
    class Seed {
        int seed;
        Seed(int seed) { this.seed = seed; }
        int grow(int more) { return seed + more; }
    }
    public static void main(String[] args) {
        Wrap w = new Wrap();
        System.out.println(w.new Seed(2).grow(3));
    }
}
