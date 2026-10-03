public class MultilevelPairRunner {
    public static void main(String[] a) {
        BR2$Mid m = new BR2$Base2();
        System.out.println(m.next().getClass().getName());
        BR2$Node2 n = m;
        System.out.println(n.next().getClass().getName());
    }
}
