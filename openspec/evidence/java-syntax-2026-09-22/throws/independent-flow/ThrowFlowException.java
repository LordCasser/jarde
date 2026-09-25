public class ThrowFlowException extends RuntimeException {
 public ThrowFlowException(String text){super(text);ThrowFlowEffects.calls=ThrowFlowEffects.calls*10+2;if(ThrowFlowEffects.mode==2)throw ThrowFlowEffects.FAIL;}
}
