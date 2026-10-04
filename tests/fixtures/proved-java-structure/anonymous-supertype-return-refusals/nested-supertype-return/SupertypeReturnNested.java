class Holder {
    interface Marker {
        String tag();
    }
}

abstract class Base implements Holder.Marker {
    abstract String extra();
}

public final class SupertypeReturnNested {
    // Negative (recover-anonymous-supertype-return tasks 1.3d): the declared return type is the
    // nested `Holder.Marker`, whose binary name `Holder$Marker` contains `$`. It sits in the
    // direct superclass's own interfaces (one layer, so the hierarchy fact alone would admit
    // it), but a `$` name is not directly spellable source text at the root class's package —
    // the existing `anonymous_super_source_type_unproved` refusal must keep firing. This slice
    // does not widen that check (the `anonymous-capture` shape stays out of scope).
    static Holder.Marker create() {
        return new Base() {
            @Override
            String extra() {
                return "extra";
            }

            @Override
            public String tag() {
                return "tagged";
            }
        };
    }

    public static void main(String[] args) {
        System.out.println(create().tag());
    }
}
