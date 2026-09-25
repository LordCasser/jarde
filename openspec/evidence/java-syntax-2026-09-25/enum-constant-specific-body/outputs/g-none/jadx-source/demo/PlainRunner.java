package demo;

/* JADX INFO: loaded from: input-g-none.jar:demo/PlainRunner.class */
public class PlainRunner {
    public static void main(String[] strArr) {
        for (Plain plain : Plain.values()) {
            System.out.println(plain.name() + "=" + plain.label() + ":runtimeIsEnum=" + (plain.getClass() == Plain.class));
        }
    }
}
