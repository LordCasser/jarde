public class GuardResource implements AutoCloseable {public GuardResource(){GuardEffects.step(5);}public void close(){GuardEffects.step(4);}}
