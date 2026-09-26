public class OuterSuperEffects extends EffectsBase {
    static int events;

    static int tick(int value, boolean fail) {
        events = events * 10 + value;
        if (fail) {
            throw new IllegalStateException("tick " + value);
        }
        return value;
    }

    @Override
    int combine(int left, int right) {
        events = events * 10 + 9;
        return 99;
    }

    class Member {
        int run(boolean failFirst) {
            return OuterSuperEffects.super.combine(tick(1, failFirst), tick(2, false));
        }
    }

    public static void main(String[] args) {
        OuterSuperEffects outer = new OuterSuperEffects();
        Member member = outer.new Member();
        System.out.println(member.run(false) + ":" + events);
        events = 0;
        try {
            member.run(true);
            System.out.println("missing exception");
        } catch (IllegalStateException expected) {
            System.out.println("fail:" + events);
        }
    }
}
