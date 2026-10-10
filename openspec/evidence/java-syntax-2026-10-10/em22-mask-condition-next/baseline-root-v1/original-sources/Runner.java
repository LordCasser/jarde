public class Runner {
    public static void main(String[] args) {
        MaskCondition value = new MaskCondition();
        System.out.println(value.method3(3, 4));
        System.out.println(value.method3(14, 6));
        System.out.println(value.method3(8, 7));
        System.out.println(value.method3(-6, 12));
        System.out.println(value.method3(Integer.MAX_VALUE, 1));
    }
}
