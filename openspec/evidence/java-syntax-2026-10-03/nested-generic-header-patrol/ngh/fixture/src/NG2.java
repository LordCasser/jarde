public class NG2 {
    static class Pair<U, V> { U first; V second; }
    public static void main(String[] a) {
        Pair<String, Integer> pair = new Pair<String, Integer>();
        pair.first = "a";
        pair.second = 1;
        System.out.println(pair.first + " " + pair.second);
    }
}
