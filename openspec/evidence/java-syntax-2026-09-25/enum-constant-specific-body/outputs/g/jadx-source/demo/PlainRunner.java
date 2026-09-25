package demo;

/* JADX INFO: loaded from: input-g.jar:demo/PlainRunner.class */
public class PlainRunner {
    public static void main(String[] args) {
        for (Plain value : Plain.values()) {
            System.out.println(value.name() + "=" + value.label() + ":runtimeIsEnum=" + (value.getClass() == Plain.class));
        }
    }
}
