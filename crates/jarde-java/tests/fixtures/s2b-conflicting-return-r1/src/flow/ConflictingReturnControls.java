package flow;

import identity.Foo;
import java.util.List;

public final class ConflictingReturnControls {
    @SuppressWarnings({"unchecked", "rawtypes"})
    public static List<Foo> conflict(List<String> source) {
        return (List<Foo>) (List) source;
    }
}
