import { Elements, PaymentElement } from "@stripe/react-stripe-js";
import { loadStripe, Stripe } from "@stripe/stripe-js";
import { useEffect, useRef, useState } from "react";
import { backendUrlPrefix } from "../config";

export default function Card() {
    return <>
        <p>You mine CardToken by using a credit card or a bank account (unlike Bitcoin that is mined by costly equipment).</p>
        <p>To mine an amount of CardToken corresponding to a certain amount of money, pay any amount of money
            to your account 
            first your account will be anonymously stored in our database and then you pay.
            After you paid, our system will initiate crypto transfer to your account.
        </p>
        <PaymentForm/>
    </>
}

function PaymentForm(userAddress) {
    const [options, setOptions] = useState(null as unknown as object);
    const [stripePromise, setStripePromise] = useState(null as Promise<Stripe | null> | null);
    const userAccount = useRef(null);
    useEffect(() => {
        async function doIt() {
            const stripe_pubkey = await (await fetch(backendUrlPrefix + "/stripe-pubkey")).text();
            // TODO: If `fiat_amount` is too small, "no payment methods" error.
            const res = await (await fetch(backendUrlPrefix + "/create-payment-intent?fiat_amount=1099")).json(); // FIXME
            const client_secret: string = res["client_secret"];

            const stripePromise_: Promise<Stripe | null> = loadStripe(stripe_pubkey);

            setOptions({
                clientSecret: client_secret,
                appearance: {},
            });
            setStripePromise(stripePromise_);
        }
        doIt();
    }, []);

    return (
        <Elements stripe={stripePromise} options={options}>
            <form>
                <p>
                    <label htmlFor="userAccount">Your crypto account:</label> {" "}
                    <input type="text" id="userAccount" ref={userAccount}/> {" "}
                    <label htmlFor="fiatAmount">Investment, in USD:</label> {" "}
                    <input type="number" id="fiatAmount" ref={userAccount}/>
                </p>
                <PaymentElement />
                <p><button>Submit</button></p>
            </form>
        </Elements>
    );
}


// async function initiatePayment() {
//     const userAddress = document.getElementById('userAccount');
//     await doInitiatePayment(userAddress);
// }
