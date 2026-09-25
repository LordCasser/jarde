package defpackage;

/* JADX INFO: loaded from: anonymous-probe.jar:AnonymousProbe.class */
public class AnonymousProbe {
    private int state = 2;

    /* JADX INFO: loaded from: anonymous-probe.jar:AnonymousProbe$Action.class */
    public interface Action {
        int run(int i);
    }

    public Action make(int i) {
        final int i2 = i + 1;
        return new Action(this) { // from class: AnonymousProbe.1
            final /* synthetic */ AnonymousProbe this$0;

            {
                this.this$0 = this;
            }

            @Override // AnonymousProbe.Action
            public int run(int i3) {
                return i2 + this.this$0.state + i3;
            }
        };
    }

    public static void main(String[] strArr) {
        System.out.println(new AnonymousProbe().make(3).run(4));
    }
}
