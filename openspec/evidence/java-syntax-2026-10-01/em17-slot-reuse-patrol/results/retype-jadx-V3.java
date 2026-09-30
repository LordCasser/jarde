
/* JADX INFO: loaded from: V3.class */
public class V3 {
    public static Object join(boolean z) {
        return z ? new int[]{1, 2, 3} : new boolean[]{true};
    }

    public static void main(String[] strArr) {
        System.out.println(join(true).getClass().getName());
    }
}
