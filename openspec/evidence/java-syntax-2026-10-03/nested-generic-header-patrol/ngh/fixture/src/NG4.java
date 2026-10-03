public class NG4 {
    static class Box<U> { U value; }
    public static void main(String[] a) {
        Box<String> box = new Box<String>();
        box.value = "y";
        System.out.println(box.value);
    }
}
