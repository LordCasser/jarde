package p;

public class Check {
    public static void main(String[] args) throws Exception {
        Class.forName("p.package-info");
        System.out.println(Check.class.getPackage().isAnnotationPresent(Deprecated.class));
    }
}
