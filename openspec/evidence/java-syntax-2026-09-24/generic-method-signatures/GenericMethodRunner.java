public class GenericMethodRunner {
    public static void main(String[] args) throws Exception {
        System.out.println(GenericMethodProbe.choose(Integer.valueOf(3), Integer.valueOf(4), true));
        System.out.println(GenericMethodProbe.class.getDeclaredMethod("choose", Number.class, Number.class, boolean.class).getTypeParameters().length);
    }
}
