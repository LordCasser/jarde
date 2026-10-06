import java.util.*;

/// The change's three negative probes: a bound receiver whose capture is not an allocation
/// (parameter read, field read) and one whose local slot is stored again after the capture.
/// Each keeps the creation-time refusal verbatim — the timing difference the refusal names is
/// real in all three, so none may be adapted.
public class BRN {
    StringBuilder buf = new StringBuilder();

    static String nullableParameter(Optional<String> o, StringBuilder sb) {
        o.ifPresent(sb::append);
        return sb.toString();
    }

    String nullableField(Optional<String> o) {
        o.ifPresent(buf::append);
        return buf.toString();
    }

    static String rewrittenAfterCapture(Optional<String> o) {
        StringBuilder sb = new StringBuilder();
        o.ifPresent(sb::append);
        sb = new StringBuilder("z");
        return sb.toString();
    }
}
