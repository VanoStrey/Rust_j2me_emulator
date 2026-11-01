package fixtures;

import javax.microedition.midlet.MIDlet;
import javax.microedition.lcdui.Display;
import javax.microedition.lcdui.Graphics;
import javax.microedition.lcdui.game.GameCanvas;

public class Fixture3RegionalFlush extends MIDlet {
    public static class TestCanvas extends GameCanvas {
        public TestCanvas() {
            super(false);
        }
        public void runTest() {
            Graphics g = getGraphics();
            g.setColor(0x00000000);
            g.fillRect(0, 0, 240, 320);
            // Draw box 1: (10, 10, 20, 20) in Blue
            g.setColor(0x000000FF);
            g.fillRect(10, 10, 20, 20);
            // Draw box 2: (50, 50, 20, 20) in Red
            g.setColor(0x00FF0000);
            g.fillRect(50, 50, 20, 20);
            // Flush only box 1
            flushGraphics(10, 10, 20, 20);
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
