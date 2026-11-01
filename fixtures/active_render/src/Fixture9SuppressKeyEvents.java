package fixtures;

import javax.microedition.midlet.MIDlet;
import javax.microedition.lcdui.Display;
import javax.microedition.lcdui.game.GameCanvas;

public class Fixture9SuppressKeyEvents extends MIDlet {
    public static int keyPressedCount = 0;
    public static int lastKeyStates = 0;

    public static class SuppressedCanvas extends GameCanvas {
        public SuppressedCanvas() {
            super(true); // Suppress key callbacks!
        }
        protected void keyPressed(int keyCode) {
            keyPressedCount++;
        }
        public void pollKeys() {
            lastKeyStates = getKeyStates();
        }
    }

    protected void startApp() {
        SuppressedCanvas sc = new SuppressedCanvas();
        Display.getDisplay(this).setCurrent(sc);
    }
    protected void pauseApp() {}
    protected void destroyApp(boolean u) {}
}
