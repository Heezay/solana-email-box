use anchor_lang::{accounts::program, AccountDeserialize};
use rand::Rng;
use solana_email_box::{mailbox_pda, messenger, send_direct_mesage, Mailbox, Message};
use solana_program::instruction::Instruction;
use solana_program_test::{processor, tokio, BanksClientError, ProgramTest, ProgramTestContext};
use solana_sdk::{
    account::AccountSharedData, pubkey::Pubkey, signature::Keypair, signer::Signer,
    transaction::Transaction,
};

#[tokio::test]
async fn test_program() {
    let mut validator = ProgramTest::default();
    validator.add_program("program", messenger::ID, processor!(messenger::entry));
    let alpha = add_account(&mut validator);
    let beta = add_account(&mut validator);

    let mut context = validator.start_with_context().await;

    let alpha_mailbox = messenger::mailbox_pda(&alpha.pubkey());
    assert!(context
        .banks_client
        .get_account(alpha_mailbox)
        .await
        .unwrap()
        .is_none());

    let beta_mailbox = messenger::mailbox_pda(&beta.pubkey());
    assert!(context
        .banks_client
        .get_account(beta_mailbox)
        .await
        .unwrap()
        .is_none());

    let ciphertext: Vec<u8> = "Hello".into();
    let first_message_pda = send_direct_message(&mut context, &alpha, &beta.pubkey(), ciphertext)
        .await
        .unwrap();

    let message = context
        .banks_client
        .get_account(first_message_pda)
        .await
        .unwrap()
        .unwrap();
    let message_data = Message::try_deserialize(&mut message.data.as_ref()).unwrap();
    assert_eq!(message_data.inbox, None);
    assert_eq!(message_data.ciphertext, ciphertext);
    let chat = context
        .banks_client
        .get_account(mailbox_pda(&beta.pubkey()))
        .await
        .unwrap()
        .unwrap();
    let chat_data = Mailbox::try_deserialize(&mut chat.data.as_ref()).unwrap();
    assert_eq!(chat_data.inbox, Some(first_message_pda));

    let encrypted_response: Vec<u8> = "Hi! Who's this?".into();
    let second_message_pda = send_direct_message(
        &mut context,
        &alpha,
        beta.pubkey(),
        encrypted_response.clone(),
    )
    .await
    .unwrap();
    let message = context
        .banks_client
        .get_account(second_message_pda)
        .await
        .unwrap()
        .unwrap();
    let message_data = Message::try_deserialize(&mut message.data.as_ref()).unwrap();
    assert_eq!(message_data.inbox, Some(first_message_pda));
    assert_eq!(message_data.ciphertext, encrypted_response);
    let chat = context
        .banks_client
        .get_account(mailbox_pda(&beta.pubkey()))
        .await
        .unwrap()
        .unwrap();
    let chat_data = Mailbox::try_deserialize(&mut chat.data.as_ref()).unwrap();
    assert_eq!(chat_data.inbox, Some(second_message_pda));
}

async fn send_direct_message(
    context: &mut ProgramTestContext,
    sender: &Keypair,
    receiver: Pubkey,
    encrypted_text: Vec<u8>,
) -> Result<Pubkey, BanksClientError> {
    let from_pubkey = sender.pubkey();
    let seed: [u8; 8] = rand::thread_rng().gen();
    let (message_pda, _bump) = Pubkey::find_program_address(&[&seed], &program::ID);
    let instruction =
        send_direct_mesage(sender, receiver, seed.into(), message_pda, encrypted_text);
    execute(context, sender, &[instruction], vec![sender]).await?;
    Ok(message_pda)
}

/// Create new transaction instance, then send a transaction and return until the transaction has been finalized or rejected  
async fn execute(
    context: &mut ProgramTestContext,
    payer: &Keypair,
    instructions: &[Instruction],
    signers: Vec<&Keypair>,
) -> Result<(), BanksClientError> {
    let transaction = Transaction::new_signed_with_payer(
        instructions,
        Some(&payer.pubkey()),
        &signers,
        context.banks_client.get_latest_blockhash().await?,
    );
    context.banks_client.process_transaction(transaction).await
}

fn add_account(validator: &mut ProgramTest) -> Keypair {
    let keypair = Keypair::new();
    let account = AccountSharedData::new(1_000_000_000, 0, &solana_sdk::system_program::id());
    validator.add_account(keypair.pubkey(), account.into());
    keypair
}
