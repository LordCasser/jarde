public class ClassLiteralRunner {
    public static void main(String[] args) {
        System.out.println("reference:" + ClassLiteralProbe.reference().getName());
        System.out.println("array:" + ClassLiteralProbe.array().getName());
        System.out.println("primitiveArray:" + ClassLiteralProbe.primitiveArray().getName());
        System.out.println("self:" + (ClassLiteralProbe.self() == ClassLiteralProbe.class));
        System.out.println("primitive:" + ClassLiteralProbe.primitive().getName());
        System.out.println("void:" + ClassLiteralProbe.voidType().getName());
        ClassLiteralProbe.calls = 0;
        System.out.println("argument:" + ClassLiteralProbe.argument().getName()
                + ":calls=" + ClassLiteralProbe.calls);
    }
}
