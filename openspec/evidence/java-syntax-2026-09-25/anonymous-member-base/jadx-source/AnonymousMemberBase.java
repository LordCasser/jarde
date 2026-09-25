package defpackage;

import java.util.Objects;

/* JADX INFO: loaded from: AnonymousMemberBase.class */
public final class AnonymousMemberBase {
    private static final StringBuilder EVENTS = new StringBuilder();

    /* JADX INFO: loaded from: AnonymousMemberBase$Outer.class */
    static class Outer {
        Outer() {
        }

        /* JADX INFO: loaded from: AnonymousMemberBase$Outer$Base.class */
        class Base {
            final int value;

            Base(int value) {
                this.value = value;
                AnonymousMemberBase.event("base(" + value + ")");
            }

            int render() {
                return this.value;
            }
        }
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static void event(String value) {
        if (EVENTS.length() > 0) {
            EVENTS.append(',');
        }
        EVENTS.append(value);
    }

    private static Outer outer() {
        event("outer");
        return new Outer();
    }

    private static Outer nullOuter() {
        event("nullOuter");
        return null;
    }

    private static int sideEffect() {
        event("argument");
        return 7;
    }

    private static Outer.Base normal() {
        Outer outer = outer();
        Objects.requireNonNull(outer);
        return new Outer.Base(outer, sideEffect()) { // from class: AnonymousMemberBase.1
            /* JADX WARN: 'super' call moved to the top of the method (can break code semantics) */
            {
                super(value);
                Objects.requireNonNull(outer);
            }

            @Override // AnonymousMemberBase.Outer.Base
            int render() {
                AnonymousMemberBase.event("anonymous");
                return this.value + 1;
            }
        };
    }

    private static Outer.Base nullPath() {
        Outer outer = nullOuter();
        Objects.requireNonNull(outer);
        return new 2(outer, sideEffect());
    }

    public static void main(String[] args) {
        Outer.Base value = normal();
        System.out.println("normal=" + value.render() + ";events=" + ((Object) EVENTS));
        EVENTS.setLength(0);
        try {
            nullPath();
            throw new AssertionError("expected null receiver failure");
        } catch (NullPointerException e) {
            System.out.println("null=" + ((Object) EVENTS));
        }
    }
}
