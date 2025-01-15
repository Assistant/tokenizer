# tokenizer
This application lets you generate a Twitch OAuth token, which is needed for tools like chat bots or other Twitch integrations. It’s designed for those who don’t have a development setup but still want a secure way to authorize their account without relying on third-party services. Your token stays private and is authorized directly through Twitch.

## Usage
1. **Create a Twitch Application**

   Follow the steps in [Creating an Application](#creating-an-application) to set up your Twitch application.

2. **Run tokenizer**

3. **Enter Required Details**
   * **Client ID** and **Client Secret**: Input the credentials from your Twitch application.
   * **Scopes**: Provide the necessary scopes for your integration, separated by spaces. For example, `chat:read chat:edit`.

4. **Authorize in Your Browser**

   The application will open a web browser for OAuth authorization. Verify that the requesting application is the one you created.

5. **Reuse or Edit Settings**

   On subsequent runs, tokenizer will remember your previous input.
   To update settings, manually edit the configuration files listed.

## Creating an Application
1. Log in to the [Twitch's developer console](https://dev.twitch.tv/console/apps/create).
2. Fill in the following details:
   * **Name**: Your choice (must be unique)
   * **OAuth Redirect URLs**: `http://localhost:3000`
   * **Category**: Your choice, Other is fine
   * **Client Type**: Confidential
3. After creating the application, click `Manage` next to it.
   * Note down the **Client ID**
   * Generate and save the **Client Secret** by clicking `New Secret`.

## Aditional Information
**Security**: All data provided to tokenizer remains on your computer and is only sent to Twitch during the OAuth process.

**Port Configuration**: Use the `PORT` environment variable to change the listening port. Ensure this matches the **OAuth Redirect URLs** in your Twitch application.

