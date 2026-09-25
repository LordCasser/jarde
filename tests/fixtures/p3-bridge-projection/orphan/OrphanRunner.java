import java.lang.invoke.MethodHandles;
import java.lang.invoke.MethodType;

public final class OrphanRunner {
    public static void main(String[] args) throws Throwable {
        OrphanBridge subject = new OrphanBridge();
        Object forwarded = MethodHandles.lookup()
            .findVirtual(OrphanBridge.class, "get", MethodType.methodType(Object.class))
            .invoke(subject);
        System.out.println(subject.get() + "|" + forwarded);
    }
}
