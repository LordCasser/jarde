package demo;

/* JADX INFO: loaded from: input-g.jar:demo/MixedRunner.class */
public class MixedRunner {
    public static void main(String[] args) {
        for (Mixed value : Mixed.values()) {
            System.out.println(value.name() + ":" + value.ordinal() + ":" + value.value() + ":runtimeIsEnum=" + (value.getClass() == Mixed.class) + ":class=" + value.getClass().getName() + ":declaring=" + value.getDeclaringClass().getSimpleName());
        }
    }
}
