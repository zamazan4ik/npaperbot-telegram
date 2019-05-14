#include <CLI/CLI.hpp>

#include <tgbot/net/CurlHttpClient.h>
#include <tgbot/tgbot.h>

#include <nlohmann/json.hpp>

#include <boost/algorithm/string.hpp>

#include <chrono>
#include <cstdlib>
#include <iostream>
#include <mutex>
#include <string>
#include <thread>
#include <vector>

std::mutex papersDatabase;

void updatePapersDatabase(nlohmann::json& papers)
{
    std::vector<TgBot::HttpReqArg> args;
    TgBot::CurlHttpClient httpClient;

    static const std::string dbAddress = "https://raw.githubusercontent.com/wg21link/db/master/index.json";
    const TgBot::Url uri(dbAddress);

    const std::string result = httpClient.makeRequest(uri, args);

    std::lock_guard<std::mutex> lockGuard(papersDatabase);
    papers = nlohmann::json::parse(result);
}

int main(int argc, char* argv[])
{
    CLI::App app("nPaperBot Telegram");

    std::string token;
    app.add_option("--token", token, "Telegram Bot API token")->required();

    int MaxResultCount = 20;
    app.add_option("--max-results-count", MaxResultCount, "Maximum results count per request");

    int MaxMessageLength = 2500;
    app.add_option("--max-message-length", MaxMessageLength, "Maximum result message length");

    CLI11_PARSE(app, argc, argv);

    nlohmann::json papers;
    updatePapersDatabase(papers);

    std::thread updatePapersThread([&papers]()
        {
            while(true)
            {
                using namespace std::chrono_literals;
                std::this_thread::sleep_for(std::chrono::duration(10min));

                updatePapersDatabase(papers);
            }
        });
    updatePapersThread.detach();

    TgBot::Bot bot(token);
    bot.getEvents().onCommand("paper", [&bot, &papers, MaxResultCount, MaxMessageLength](TgBot::Message::Ptr message)
        {
            std::string fixedMessage = message->text.substr();

            fixedMessage.erase(fixedMessage.begin(), fixedMessage.begin() + fixedMessage.find(' ') + 1);

            const std::string ResultFiller = "For the request \"" +  fixedMessage + "\":\n";
            std::string result = ResultFiller;
            bool anyResult = false;
            int resultCount = 0;

            std::lock_guard<std::mutex> lockGuard(papersDatabase);
            for(auto const& paper : papers)
            {
                if(paper.find("type") == paper.end() || paper.find("title") == paper.end() ||
                    paper.find("author") == paper.end() || paper.find("link") == paper.end() ||
                    paper["type"].get<std::string>() != "paper")
                {
                    continue;
                }

                const auto paperTitle = paper["title"].get<std::string>();
                if(boost::algorithm::icontains(paperTitle, fixedMessage))
                {
                    if(resultCount == MaxResultCount)
                    {
                        result += "There are more papers. Please use more precise query.";
                        break;
                    }

                    anyResult = true;
                    ++resultCount;
                    result += paper["title"].get<std::string>() + " from " +
                              paper["author"].get<std::string>() + "\n" + paper["link"].get<std::string>() + "\n\n";

                    if(result.size() > MaxMessageLength)
                    {
                        bot.getApi().sendMessage(message->chat->id, result);
                        result = ResultFiller;
                    }
                }
            }

            if(!anyResult)
            {
               result +=  "Found nothing. Sorry.";
            }

            if(result != ResultFiller)
            {
                bot.getApi().sendMessage(message->chat->id, result);
            }
        });

    bot.getEvents().onCommand("help", [&bot](TgBot::Message::Ptr message)
    {
        bot.getApi().sendMessage(message->chat->id, "Use \"/paper\" command with substring from a proposal title."
                                                    "Search works only for titles. Fuzzy search isn't supported yet.");
    });

    try
    {
        std::cout << "Bot username: " << bot.getApi().getMe()->username << std::endl;
        TgBot::TgLongPoll longPoll(bot);
        while (true)
        {
            std::cout << "Long poll started\n";
            longPoll.start();
        }
    }
    catch (const TgBot::TgException& e)
    {
        std::cout << "Telegram bot exception: " << e.what() << std::endl;
    }
    catch(const std::exception& e)
    {
        std::cout << "Exception: " << e.what() << std::endl;
    }
    return 0;
}