public class GuardInlineControls {
 public static void locked(){synchronized(GuardEffects.lock()){GuardEffects.mark();}}
 public static int returned(){synchronized(GuardEffects.shared){return GuardEffects.value();}}
 public static void resource(){try(GuardResource r=new GuardResource()){GuardEffects.mark();}}
}
