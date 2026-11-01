package fixtures;

import javax.microedition.midlet.MIDlet;
import javax.microedition.lcdui.Display;
import javax.microedition.lcdui.Graphics;
import javax.microedition.lcdui.game.GameCanvas;

public class Fixture2FullFlush extends MIDlet {
    public static class TestCanvas extends GameCanvas {
        public TestCanvas() {
            super(false);
        }
        public void runTest() {
            Graphics g = getGraphics();
            g.setColor(0x0000FF00); // Green
            g.fillRect(10, 10, 20, 20);
            flushGraphics();
        }
    }

    protected void startApp() {
        TestCanvas c = new TestCanvas();
        Display.getDisplay(this).setCurrent(c);
        c.runTest();
    }
    protected void pauseApp() {}
    protected void destroyApp(boolean u) {}
}
