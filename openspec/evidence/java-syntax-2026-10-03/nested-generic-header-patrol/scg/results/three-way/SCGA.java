public class SCGA<T extends java.lang.Comparable<T>> {
    public void note(T value) {
        java.util.Collections.singletonList(value);
        return;
    }
    public static void main(java.lang.String[] args) {
        SCGA z = new SCGA<java.lang.String>();
        java.lang.Comparable word = "b";
        z.note(word);
        System.out.println("note:" + word.equals("b"));
    }
}
