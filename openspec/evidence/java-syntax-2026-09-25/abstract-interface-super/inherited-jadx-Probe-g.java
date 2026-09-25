package defpackage;

/* JADX INFO: loaded from: inherited.jar:Probe.class */
public final class Probe implements Child {
    @Override // defpackage.Parent
    public int value() {
        return super.value();
    }

    public static void main(String[] args) {
        System.out.println(new Probe().value());
    }
}
