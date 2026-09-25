public class OrderValue {
 public int value;
 public OrderValue(){DeferredEffects.trace=DeferredEffects.trace*10+1;if(DeferredEffects.mode==1)throw DeferredEffects.FAILURE;}
}
