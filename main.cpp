#include "tgbot/net/CurlHttpClient.h"
#include "tgbot/tgbot.h"

#include "nlohmann/json.hpp"

#include <chrono>
#include <iostream>
#include <mutex>
#include <string>
#include <thread>

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

    TgBot::Bot bot(argv[1]);
    bot.getEvents().onCommand("paper", [&bot, &papers](TgBot::Message::Ptr message)
        {
            std::string fixedMessage = message->text.substr();

            fixedMessage.erase(fixedMessage.begin(), fixedMessage.begin() + fixedMessage.find(' ') + 1);

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
                if(paperTitle.find(fixedMessage) != std::string::npos)
                {
                    bot.getApi().sendMessage(message->chat->id, paper["title"].get<std::string>() + " from " +
                            paper["author"].get<std::string>() + "\n" + paper["link"].get<std::string>());
                }
            }
        });

    bot.getEvents().onCommand("help", [&bot, &papers](TgBot::Message::Ptr message)
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