package p;

/* JADX INFO: loaded from: original.jar:p/Subject.class */
public class Subject {
    public static int value;

    public static Base make() {
        return new Base() { // from class: p.Subject.1
            {
                Subject.value = 1;
            }

            @Override // p.Base
            public void run() {
                Subject.value += 7;
            }
        };
    }
}
