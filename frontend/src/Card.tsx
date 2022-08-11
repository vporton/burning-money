import { Elements, PaymentElement, useElements, useStripe } from "@stripe/react-stripe-js";
import { loadStripe, PaymentIntent, Stripe, StripeElements } from "@stripe/stripe-js";
import { FormEvent, RefObject, useEffect, useRef, useState } from "react";
import { backendUrlPrefix } from "./config";
import React from 'react';
import { NavLink } from "react-router-dom";
import { EthAddress } from "./components/EthAddress";
import { Kyc } from './KYC';
import { ethers } from 'ethers';
const { utils, BigNumber: BN } = ethers;

export default function Card(props: { bidDay: number }) {
    const [user, setUser] = useState<string | null>(null);
    const [passedKyc, setPassedKyc] = useState(true); // don't show by default
    fetch(backendUrlPrefix + "/identity", {credentials: 'include'})
        .then(u => u.json())
        .then(u => {
            setUser(u.id);
            setPassedKyc(u.kyc);
        });
    function logout() {
        window.fetch(backendUrlPrefix + "/logout", {
            method: "POST",
            credentials: 'include',
        })
            .then(() => setUser(null)); // TODO: Handle errors.
    }

    return <>
        {user === null ?
            <>
                <NavLink to={'/login'}>Login</NavLink> <NavLink to={'/register'}>Register</NavLink>
            </> :
            <>
                <a href='#' onClick={logout}>Logout</a>
            </>
        }
        {passedKyc ? "" : <Kyc/>}
        <p>You mine CardToken by using a credit card or a bank account (unlike Bitcoin that is mined by costly equipment).</p>
        <p>To mine an amount of CardToken corresponding to a certain amount of money, pay any amount of money.</p>
        {user !== null ? <PaymentForm bidDay={props.bidDay}/> : ""}
    </>
}

// https://stripe.com/docs/payments/finalize-payments-on-the-server

function PaymentForm(props: { bidDay: number }) {
    const [options, setOptions] = useState(null as unknown as object);
    const [stripePromise, setStripePromise] = useState(null as Promise<Stripe | null> | null);
    const [fiatAmount, setFiatAmount] = useState(0);
    const [fiatAmountRaw, setFiatAmountRaw] = useState("");
    const [cryptoAmount, setCryptoAmount] = useState("0");
    const [showPayment, setShowPayment] = useState(false);
    const [showingPayment, setShowingPayment] = useState(false);
    const [showPaymentError, setShowPaymentError] = useState("");
    const [paymentIntentId, setPaymentIntentId] = useState("");
    const [userAccount, setUserAccount] = useState("");
    const [ethAddrValid, setEthAddrValid] = useState(false);
    const [serverAccount, setServerAccount] = useState("");
    const [serverBalance, setServerBalance] = useState(BN.from(0));

    // TODO: duplicate code
    function handleAccountsChanged(accounts: any) {
        if(accounts[0]) {
            setUserAccount(accounts[0]);
        }
    }

    useEffect(() => {
        // TODO: duplicate code
        (window as any).ethereum
            .request({ method: 'eth_accounts' })
            .then(handleAccountsChanged)
            .catch((err: any) => {
                console.error(err);
            });
        (window as any).ethereum.on('accountsChanged', handleAccountsChanged);
    }, []);


    async function createPaymentIntent() {
        const res = await (await fetch(`${backendUrlPrefix}/create-payment-intent?fiat_amount=${fiatAmount}`, {
            method: "POST",
            credentials: 'include',
        })).json();

        if (res.error) {
            setShowPaymentError(res.error.message);
            setShowPayment(false);
        } else {
            const clientSecret: string = res["client_secret"];
            const paymentIntentId: string = res["id"];

            setOptions({
                clientSecret,
                appearance: {},
            });
            setPaymentIntentId(paymentIntentId);
        }
    }

    async function doShowPayment() {
        setShowingPayment(true);
        await createPaymentIntent();
        setShowPayment(true);
    }

    useEffect(() => {
        async function doIt() {
            await (window as any).ethereum.enable();
            const provider = new ethers.providers.Web3Provider((window as any).ethereum, "any");
            provider.on("block", (tx) => {
                if(serverAccount !== "") {
                    provider.getBalance(serverAccount)
                        .then(balance => setServerBalance(balance));
                }
            });
            if(serverAccount !== "") {
                provider.getBalance(serverAccount)
                    .then(balance => setServerBalance(balance));
            }
        }
        doIt();
    }, [serverAccount])

    useEffect(() => {
        fetch(backendUrlPrefix + "/stripe-pubkey")
            .then(res => res.text())
            .then(stripePubkey => {
                const stripePromise_: Promise<Stripe | null> = loadStripe(stripePubkey, {
                    betas: ['server_side_confirmation_beta_1'],
                    apiVersion: '2020-08-27;server_side_confirmation_beta=v1',
                });
                setStripePromise(stripePromise_);
            });
        fetch(backendUrlPrefix + "/server-account")
            .then(res => res.text())
            .then(serverAccount_ => {
                setServerAccount(serverAccount_);
            });
    }, []);

    function onPayClicked() {
        setPaymentIntentId("");
    }

    function setFiatAmountFromInput(input: HTMLInputElement) {
        setFiatAmount(Math.floor(0.5 + (Number(input.value) * 100)))
    }

    useEffect(() => {
        const fiatAmount_ = Math.floor(0.5 + (Number(fiatAmountRaw) * 100));
        setFiatAmount(fiatAmount_);
        fetch(`${backendUrlPrefix}/fiat-to-crypto?fiat_amount=${encodeURIComponent(fiatAmount_)}`)
            .then(res => res.text())
            .then(wei => setCryptoAmount(wei));
    }, [fiatAmountRaw])

    return (
        <>
            <p>
                <label htmlFor="userAccount">Your crypto account:</label> {" "}
                <EthAddress value={userAccount} onChange={(e: any) => setUserAccount(e.target.value) } onValid={setEthAddrValid}/> {" "}
                <label htmlFor="fiatAmount">Investment, in USD:</label> {" "}
                <input type="number" id="fiatAmount"
                    onChange={e => setFiatAmountRaw(e.target.value)} disabled={showingPayment}
                    className={/^[0-9]+(\.[0-9]+)?$/.test(String(fiatAmountRaw)) ? "" : "error"}/> {" "}
                bid: {utils.formatEther(BN.from(cryptoAmount))} GMLR* (of {utils.formatEther(serverBalance)} GLMR server balance) {" "}
                <button disabled={!ethAddrValid || fiatAmount < 0.5 || !stripePromise || showingPayment} onClick={e => doShowPayment()}
                >Next &gt;&gt;</button>
                {showingPayment ?
                    <>
                        {" "}
                        <button onClick={() => { setShowingPayment(false); setShowPayment(false); createPaymentIntent(); }}>&lt;&lt; Back</button>
                    </>
                    : ""}
            </p>
            <p>* As it was the last time retrieved.</p>
            {showPayment && <Elements stripe={stripePromise} options={options}>
                <PaymentFormContent paymentIntentId={paymentIntentId} userAccount={userAccount} bidDay={props.bidDay} onPayClicked={onPayClicked}
                onPaid={() => {setShowingPayment(false); setShowPayment(false);}}/>
            </Elements>}
            {!showPayment && <p>{showPaymentError}</p>}
        </>
    );
}

function PaymentFormContent(props: any) { // TODO: `any`
    const stripe = useStripe() as Stripe;
    const elements = useElements() as StripeElements;

    async function submitHandler(event: FormEvent<HTMLFormElement>) {
        event.preventDefault();
      
        const handleServerResponse = async (response: any) => {
            if (response.error) {
                alert(response.error); // FIXME
            } else if (response.requires_action) {
                // Use Stripe.js to handle the required next action
                const {
                    error: errorAction,
                    paymentIntent
                } = await (stripe as any).handleNextAction({
                    clientSecret: response.payment_intent_client_secret
                });

                if (errorAction) {
                    alert(errorAction); // TODO
                } else {
                    alert("Success."); // FIXME
                }
            } else {
                props.onPaid();
                alert("You've paid."); // TODO
            }
          }
        
        const stripePaymentMethodHandler = function (result: any) {
            if (result.error) {
                alert(result.error); // TODO
            } else {
                // Otherwise send paymentIntent.id to your server
                let data = [];
                data.push('payment_intent_id=' + encodeURIComponent(result.paymentIntent.id));
                data.push('crypto_account=' + encodeURIComponent(props.userAccount));
                data.push('bid_day=' + encodeURIComponent(props.bidDay));
                fetch(backendUrlPrefix + '/confirm-payment', {
                    method: 'POST',
                    credentials: 'include',
                    mode: 'cors',
                    headers: {'Content-Type': 'application/x-www-form-urlencoded'},
                    body: data.join('&'),
                }).then(function (res) {
                    return res.json();
                }).then(function (paymentResponse) {
                    handleServerResponse(paymentResponse);
                });
            }
        };

        (stripe as any).updatePaymentIntent({
            elements, // elements instance
            params: {
            //   payment_method_data: {
            //     billing_details: { ... }
            //   },
            //   shipping: { ... }
            }
        }).then(function (result: any) {
           stripePaymentMethodHandler(result)
        });
    }

    return (
        <form onSubmit={e => submitHandler(e)}> {/* FIXME: async */}
            <PaymentElement />
            <p><button>Invest</button></p> {/* TODO: Disable button for no accidental repeated clicking. */}
        </form>
    );
}

// async function initiatePayment() {
//     const userAddress = document.getElementById('userAccount');
//     await doInitiatePayment(userAddress);
// }
