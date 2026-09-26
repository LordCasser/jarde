package defpackage;

/* JADX INFO: loaded from: fixture.jar:Base.class */
class Base {
    private final String label;
    private final int value;

    Base(String str, int i) {
        this.label = str;
        this.value = i;
        AnonymousSuperArgs.event("base:" + str + ":" + i);
    }

    String render() {
        return this.label + ":" + this.value;
    }
}
