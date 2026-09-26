package p;

/* JADX INFO: loaded from: input.jar:p/Check.class */
public class Check {
    public static void main(String[] strArr) throws Exception {
        Class.forName("p.package-info");
        System.out.println(Check.class.getPackage().isAnnotationPresent(Deprecated.class));
    }
}
