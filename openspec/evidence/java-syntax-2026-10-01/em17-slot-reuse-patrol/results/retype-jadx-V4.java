
/* JADX INFO: loaded from: V4.class */
public class V4 {
    public static Object loop(int i) {
        Object obj = {1, 2, 3};
        for (int i2 = 0; i2 < i; i2++) {
            obj = new boolean[]{true};
        }
        return obj;
    }

    public static void main(String[] strArr) {
        System.out.println(loop(0).getClass().getName());
    }
}
