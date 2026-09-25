
/* JADX INFO: loaded from: positive.jar:InterfaceSuperProbe.class */
public final class InterfaceSuperProbe extends DefaultBase implements DefaultLeft, DefaultRight {
    @Override // defpackage.DefaultBase, defpackage.DefaultLeft, defpackage.DefaultRight
    public int value() {
        return super.value();
    }

    public int chooseRight() {
        return super.value();
    }

    public int both() {
        return super.value() + super.value();
    }
}
