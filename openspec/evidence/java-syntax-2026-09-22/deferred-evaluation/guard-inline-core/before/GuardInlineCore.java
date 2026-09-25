public class GuardInlineCore {
 public static void locked(){synchronized(GuardEffects.lock()){GuardEffects.mark();}}
 public static int returned(){synchronized(GuardEffects.shared){return GuardEffects.value();}}
}
