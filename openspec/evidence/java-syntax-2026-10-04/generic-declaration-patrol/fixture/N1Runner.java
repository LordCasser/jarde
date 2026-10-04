import java.util.*;
public class N1Runner {
    public static void main(String[] a) throws Exception {
        N1 n = new N1();
        System.out.println("newRet=" + N1.class.getMethod("newRet").getGenericReturnType());
        System.out.println("paramEcho=" + N1.class.getMethod("paramEcho", List.class).getGenericReturnType());
        List<String> got = n.newRet();
        got.add("x");
        System.out.println("size=" + got.size());
    }
}
