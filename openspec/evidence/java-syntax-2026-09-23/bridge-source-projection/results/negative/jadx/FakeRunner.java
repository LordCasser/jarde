package defpackage;
import java.lang.invoke.MethodHandles;
import java.lang.invoke.MethodType;

public final class FakeRunner {
    public static void main(String[] args) throws Throwable {
        FakeBridge subject = new FakeBridge();
        Object forwarded = MethodHandles.lookup()
            .findVirtual(FakeBridge.class, "get", MethodType.methodType(Object.class))
            .invoke(subject);
        System.out.println(forwarded + "|" + subject.get() + "|" + FakeBridge.count());
    }
}
