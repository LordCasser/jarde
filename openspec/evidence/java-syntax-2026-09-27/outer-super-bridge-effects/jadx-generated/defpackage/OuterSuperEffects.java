package defpackage;

import java.util.Objects;

/* JADX INFO: loaded from: new-input.jar:OuterSuperEffects.class */
public class OuterSuperEffects extends EffectsBase {
    static int events;

    static int tick(int value, boolean fail) {
        events = (events * 10) + value;
        if (fail) {
            throw new IllegalStateException("tick " + value);
        }
        return value;
    }

    /* JADX INFO: Access modifiers changed from: package-private */
    @Override // defpackage.EffectsBase
    public int combine(int left, int right) {
        events = (events * 10) + 9;
        return 99;
    }

    /* JADX INFO: loaded from: new-input.jar:OuterSuperEffects$Member.class */
    class Member {
        Member() {
        }

        int run(boolean failFirst) {
            return OuterSuperEffects.super.combine(OuterSuperEffects.tick(1, failFirst), OuterSuperEffects.tick(2, false));
        }
    }

    public static void main(String[] args) {
        OuterSuperEffects outer = new OuterSuperEffects();
        Objects.requireNonNull(outer);
        Member member = outer.new Member();
        System.out.println(member.run(false) + ":" + events);
        events = 0;
        try {
            member.run(true);
            System.out.println("missing exception");
        } catch (IllegalStateException e) {
            System.out.println("fail:" + events);
        }
    }
}
