public class NG5 {
    static class Box<U> { U value; }
    static Box<String> box;
    public static void main(String[] a) {
        box = new Box<String>();
        box.value = "z";
        System.out.println(box.value);
    }
}
