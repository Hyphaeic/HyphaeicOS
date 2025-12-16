import "./TESTING_DUMMY.css";
import Domain from "../core/A_Domain/Domain";
import Button_IC from "../core/Button/Button_IC";

/**
 * TESTING_DUMMY - Test component for window content navigation
 *
 * This is a navigable domain that demonstrates spatial navigation
 * between window content areas. Contains sample buttons for testing.
 */
export default function TESTING_DUMMY() {
    return (
        <Domain id="testing-dummy-nav" layoutMode="list-vertical" class="testing-dummy-domain">
            <div class="testing-dummy-content">
                <h2>Testing Domain</h2>
                <p>This window content is a navigable domain.</p>

                <div class="testing-dummy-buttons">
                    <Button_IC
                        id="test-btn-1"
                        order={0}
                        onClick={() => console.log("Test Button 1 clicked")}
                    >
                        Test Button 1
                    </Button_IC>

                    <Button_IC
                        id="test-btn-2"
                        order={1}
                        onClick={() => console.log("Test Button 2 clicked")}
                    >
                        Test Button 2
                    </Button_IC>

                    <Button_IC
                        id="test-btn-3"
                        order={2}
                        onClick={() => console.log("Test Button 3 clicked")}
                    >
                        Test Button 3
                    </Button_IC>
                </div>
            </div>
        </Domain>
    );
}
