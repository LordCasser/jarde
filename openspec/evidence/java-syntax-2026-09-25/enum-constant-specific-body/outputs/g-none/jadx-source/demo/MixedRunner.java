package demo;

/* JADX INFO: loaded from: input-g-none.jar:demo/MixedRunner.class */
public class MixedRunner {
    public static void main(String[] strArr) {
        for (Mixed mixed : Mixed.values()) {
            System.out.println(mixed.name() + ":" + mixed.ordinal() + ":" + mixed.value() + ":runtimeIsEnum=" + (mixed.getClass() == Mixed.class) + ":class=" + mixed.getClass().getName() + ":declaring=" + mixed.getDeclaringClass().getSimpleName());
        }
    }
}
